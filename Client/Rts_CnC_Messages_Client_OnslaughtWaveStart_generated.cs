using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_OnslaughtWaveStart
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.OnslaughtWaveStart); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.OnslaughtWaveStart)obj;
            //  Serialize WaveNumber
            s.Write(value.WaveNumber);
            //  Serialize WaveTime
            s.Write(value.WaveTime);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.OnslaughtWaveStart)) as Rts.CnC.Messages.Client.OnslaughtWaveStart;
            //  Deserialize WaveNumber
            s.Read(out value.WaveNumber);
            //  Deserialize WaveTime
            s.Read(out value.WaveTime);

            return value;
        }
        
    }
}
