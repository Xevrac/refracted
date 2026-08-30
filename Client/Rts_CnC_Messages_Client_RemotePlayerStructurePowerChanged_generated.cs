using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RemotePlayerStructurePowerChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RemotePlayerStructurePowerChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RemotePlayerStructurePowerChanged)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize PowerProduced
            s.Write(value.PowerProduced);
            //  Serialize PowerUsed
            s.Write(value.PowerUsed);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RemotePlayerStructurePowerChanged)) as Rts.CnC.Messages.Client.RemotePlayerStructurePowerChanged;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize PowerProduced
            s.Read(out value.PowerProduced);
            //  Deserialize PowerUsed
            s.Read(out value.PowerUsed);

            return value;
        }
        
    }
}
