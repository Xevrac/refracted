using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ShowPlayerPowerTimer
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ShowPlayerPowerTimer); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ShowPlayerPowerTimer)obj;
            //  Serialize Duration
            s.Write(value.Duration);
            //  Serialize XCoordinate
            s.Write(value.XCoordinate);
            //  Serialize YCoordinate
            s.Write(value.YCoordinate);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ShowPlayerPowerTimer)) as Rts.CnC.Messages.Client.ShowPlayerPowerTimer;
            //  Deserialize Duration
            s.Read(out value.Duration);
            //  Deserialize XCoordinate
            s.Read(out value.XCoordinate);
            //  Deserialize YCoordinate
            s.Read(out value.YCoordinate);

            return value;
        }
        
    }
}
