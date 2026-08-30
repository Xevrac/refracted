using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestCancelBuild
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestCancelBuild); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestCancelBuild)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize EntityType
            s.Write(value.EntityType);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestCancelBuild)) as Rts.CnC.Messages.Client.RequestCancelBuild;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize EntityType
            s.Read(out value.EntityType);

            return value;
        }
        
    }
}
