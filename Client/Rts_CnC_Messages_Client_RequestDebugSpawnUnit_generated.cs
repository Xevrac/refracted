using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestDebugSpawnUnit
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestDebugSpawnUnit); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestDebugSpawnUnit)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityClassName
            s.Write(value.EntityClassName);
            //  Serialize PositionToSpawn
            s.Write(value.PositionToSpawn);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestDebugSpawnUnit)) as Rts.CnC.Messages.Client.RequestDebugSpawnUnit;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityClassName
            s.Read(out value.EntityClassName);
            //  Deserialize PositionToSpawn
            s.Read(out value.PositionToSpawn);

            return value;
        }
        
    }
}
